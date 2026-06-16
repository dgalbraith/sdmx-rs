<details>
<summary>XSD contract: <code>AgencyType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="AgencyType">
		<xs:annotation>
			<xs:documentation>AgencyType defines the structure of an agency description. The contacts defined for the organisation are specific to the agency role the organisation is serving.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:restriction base="OrganisationType">
				<xs:sequence>
					<xs:element ref="common:Annotations" minOccurs="0"/>
					<xs:element ref="common:Link" minOccurs="0" maxOccurs="unbounded"/>
					<xs:element ref="common:Name" maxOccurs="unbounded"/>
					<xs:element ref="common:Description" minOccurs="0" maxOccurs="unbounded"/>
					<xs:element name="Contact" type="ContactType" minOccurs="0" maxOccurs="unbounded"/>
				</xs:sequence>
				<xs:attribute name="urn" type="common:AgencyUrnType" use="optional"/>
				<xs:attribute name="id" type="common:NCNameIDType" use="required">
					<xs:annotation>
						<xs:documentation>The id attribute holds the identification of the agency. The type of this id is restricted to the common:NCNNameIDType. This is necessary, since the agency identifier will be used as part of the name for simple types in data and metadata structure specific schemas and therefore must be compliant with the NCName type in XML Schema (see common:NCNameIDType for further details).</xs:documentation>
					</xs:annotation>
				</xs:attribute>
			</xs:restriction>
		</xs:complexContent>
	</xs:complexType>
```

</details>
