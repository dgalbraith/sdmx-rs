<details>
<summary>XSD contract: <code>AgencySchemeType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="AgencySchemeType">
		<xs:annotation>
			<xs:documentation>AgencySchemeType defines a specific type of organisation scheme which contains only maintenance agencies. The agency scheme maintained by a particular maintenance agency is always provided a fixed identifier and is never versioned. Therefore, agencies can be added or removed without have to version the scheme. Agencies schemes have no hierarchy, meaning that no agency may define a relationship with another agency in the scheme. In fact, the actual parent agency for an agency in a scheme is the agency which defines the scheme.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:restriction base="OrganisationSchemeType">
				<xs:sequence>
					<xs:element ref="common:Annotations" minOccurs="0"/>
					<xs:element ref="common:Link" minOccurs="0" maxOccurs="unbounded"/>
					<xs:element ref="common:Name" maxOccurs="unbounded"/>
					<xs:element ref="common:Description" minOccurs="0" maxOccurs="unbounded"/>
					<xs:sequence minOccurs="0" maxOccurs="unbounded">
						<xs:element ref="Agency"/>
					</xs:sequence>
				</xs:sequence>
				<xs:attribute name="urn" type="common:AgencySchemeUrnType" use="optional"/>
				<xs:attribute name="id" type="common:IDType" use="required" fixed="AGENCIES"/>
			</xs:restriction>
		</xs:complexContent>
	</xs:complexType>
```

</details>
