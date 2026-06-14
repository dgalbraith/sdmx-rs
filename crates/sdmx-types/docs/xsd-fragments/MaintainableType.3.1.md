<details>
<summary>XSD contract: <code>MaintainableType</code> (SDMX 3.1)</summary>

```xml
	<xs:complexType name="MaintainableType" abstract="true">
		<xs:annotation>
			<xs:documentation>MaintainableType is an abstract base type for all maintainable objects.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:extension base="MaintainableBaseType">
				<xs:attribute name="agencyID" type="NestedNCNameIDType" use="required">
					<xs:annotation>
						<xs:documentation>The agencyID must be provided and identifies the maintenance agency of the object.</xs:documentation>
					</xs:annotation>
				</xs:attribute>
				<xs:attribute name="isPartialLanguage" type="xs:boolean" use="optional" default="false">
					<xs:annotation>
						<xs:documentation>The isPartialLanguage attribute, if true, indicates that the object doesn't contain the complete set of all available languages, e.g., when obtained as a response to a GET query that requested specific languages through the HTTP header 'Accept-Language'.</xs:documentation>
					</xs:annotation>
				</xs:attribute>
				<xs:attribute name="isExternalReference" type="xs:boolean" use="optional" default="false">
					<xs:annotation>
						<xs:documentation>The isExternalReference attribute, if true, indicates that the actual object is not defined the corresponding element, rather its full details are defined elsewhere - indicated by either the registryURL, the repositoryURL, or the structureURL. The purpose of this is so that each structure message does not have to redefine object that are already defined elsewhere. If the isExternalReference attribute is not set, then it is assumed to be false, and the object should contain the full definition of its contents. If more than one of the registryURL, the repositoryURL, and the structureURL are supplied, then the application processing the object can choose the method it finds best suited to retrieve the details of the object.</xs:documentation>
					</xs:annotation>
				</xs:attribute>
				<xs:attributeGroup ref="ExternalReferenceAttributeGroup"/>
			</xs:extension>
		</xs:complexContent>
	</xs:complexType>
```

</details>
